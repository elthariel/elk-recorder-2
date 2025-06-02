import logging

import dbus

log = logging.getLogger(__name__)

IF_OM = "org.freedesktop.DBus.ObjectManager"
IF_DRIVE = "org.freedesktop.UDisks2.Drive"
IF_BLOCK = "org.freedesktop.UDisks2.Block"
IF_FS = "org.freedesktop.UDisks2.Filesystem"
IF_PROPS = "org.freedesktop.DBus.Properties"


class DBusObj:
    def __init__(self, bus, path, iface):
        self.bus = bus
        self.path = path
        self.iface = iface
        self.obj = self.bus.get_object("org.freedesktop.UDisks2", self.path)

    def iface(self, iface=None):
        if iface is None:
            iface = self.iface

        return dbus.Interface(self.obj, iface)

    @property
    def props(self):
        return dbus.Interface(self.obj, IF_PROPS)

    def prop(self, name, iface=None):
        if iface is None:
            iface = self.iface

        return self.props.Get(iface, name)


class DBusList:
    def __init__(self, bus, iface, paths=None):
        self.bus = bus
        self.iface = iface
        self.paths = paths or list()

    def add(self, path):
        self.paths.append(path)

    def remove(self, path):
        self.paths.remove(path)

    def __contains__(self, path):
        return path in self.paths

    def __getitem__(self, path):
        if isinstance(path, str):
            if path in self.paths:
                return DBusObj(self.bus, path, self.iface)
            else:
                raise KeyError(path)
        else:
            return DBusObj(self.bus, self.paths[path], self.iface)

    def __iter_gen__(self):
        for path in self.paths:
            yield self[path]

    def __iter__(self):
        return self.__iter_gen__()
