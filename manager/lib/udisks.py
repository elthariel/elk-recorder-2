import logging, functools

import humps
import dbus

from .dbus import DBusObj, DBusList, IF_OM, IF_DRIVE, IF_BLOCK, IF_FS

log = logging.getLogger(__name__)


class UdisksMonitor:
    def __init__(self, on_mount_add=None, on_mount_remove=None):
        log.debug("Connecting to DBUS system bus")
        self.bus = dbus.SystemBus()

        self.udisks = self.bus.get_object(
            "org.freedesktop.UDisks2", "/org/freedesktop/UDisks2"
        )
        self.object_manager = dbus.Interface(self.udisks, IF_OM)
        self.on_mount_add = on_mount_add
        self.on_mount_remove = on_mount_remove

        self.refresh_dbus()
        self.connect_signals()

    def refresh_dbus(self):
        self.drives = DBusList(self.bus, IF_DRIVE)
        self.filesystems = DBusList(self.bus, IF_FS)
        self.fs_to_mount = {}
        self.mount_to_fs = {}

        objects = self.object_manager.GetManagedObjects()

        for path, ifaces in objects.items():
            if IF_DRIVE in ifaces:
                self.on_interfaces_added(path, ifaces)

        for path, ifaces in objects.items():
            if IF_FS in ifaces:
                self.on_interfaces_added(path, ifaces)

    def connect_signals(self):
        signals = ["InterfacesAdded", "InterfacesRemoved"]

        for signal in signals:
            signal = humps.pascalize(signal)
            handler_name = "on_" + humps.decamelize(signal)
            if hasattr(self, handler_name):
                log.debug(f"Registered {handler_name}")
                handler = getattr(self, handler_name)
                self.bus.add_signal_receiver(
                    handler,
                    signal_name=signal,
                    dbus_interface=self.object_manager.dbus_interface,
                )

    def on_interfaces_added(self, path, ifaces):
        log.debug(f"on_interface_added: {path}")
        # log.debug(f"on_interface_added details: {ifaces}")

        if IF_DRIVE in ifaces:
            props = ifaces[IF_DRIVE]
            if props["ConnectionBus"] == "usb":
                log.info(f"New USB drive: {path}")
                self.drives.add(path)

        if IF_BLOCK in ifaces:
            drive = ifaces[IF_BLOCK]["Drive"]
            if drive in self.drives and IF_FS in ifaces:
                log.info(f"New USB filesystem: {path}")
                self.filesystems.add(path)

                mounts = ifaces[IF_FS]["MountPoints"]
                if len(mounts) > 0:
                    self.on_mounts_updated(path, mounts)

                self.bus.add_signal_receiver(
                    functools.partial(self.on_properties_changed, path),
                    signal_name="PropertiesChanged",
                    path=path,
                )

    def on_interfaces_removed(self, path, ifaces):
        log.debug(f"on_interface_removed: {path}")
        # log.info(f"on_interface_removed details: {ifaces}")

        if path in self.drives:
            self.drives.remove(path)

        if path in self.filesystems:
            self.filesystems.remove(path)
            self.remove_mount(path)

    def on_properties_changed(self, path, iface, props, *_args):
        log.debug(f"props changed: {path}, {iface}, {props}")
        if iface == IF_FS and "MountPoints" in props:
            self.on_mounts_updated(path, props["MountPoints"])

    def on_mounts_updated(self, path, mounts):
        if len(mounts) > 0:
            mounts = [bytearray(item[:-1]).decode("utf-8") for item in mounts]
            mount = mounts[0]
            self.add_mount(path, mount)
        else:
            self.remove_mount(path)

    def add_mount(self, path, mount):
        if path in self.fs_to_mount:
            log.warn(f"Overriding mount point {self.fs_to_mount[path]} for {path}")

        self.fs_to_mount[path] = mount
        self.mount_to_fs[mount] = path

        if self.on_mount_add is not None:
            self.on_mount_add(path, mount)

    def remove_mount(self, path):
        if path in self.fs_to_mount:
            mount = self.fs_to_mount.pop(path)
            self.mount_to_fs.pop(mount)

            if self.on_mount_remove is not None:
                self.on_mount_remove(path, mount)
        else:
            log.warn(f"Trying to clear non existent mount for {path}")
