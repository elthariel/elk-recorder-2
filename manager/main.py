import sys, os, logging, functools

sys.path.append(os.path.dirname(__file__))
sys.path.append(os.path.join(os.path.dirname(__file__), "proto"))

import click, grpc

from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

from lib import UdisksMonitor, Manager

log = logging.getLogger(__name__)


def setup_logging():
    level = getattr(logging, os.getenv("ELKR_LOG_LEVEL", "DEBUG"))
    logging.basicConfig(level=level)


@click.group
def cli():
    pass


@cli.command
@click.option("--engine-address", "-e", "address", default="localhost:50051")
def watch(address):
    setup_logging()
    log.info("Starting ELKr manager")

    DBusGMainLoop(set_as_default=True)

    Manager.wait_for_channel(address)

    manager = Manager(address)

    monitor = UdisksMonitor(
        on_mount_add=manager.on_mount_add, on_mount_remove=manager.on_mount_remove
    )

    log.info("Entering main loop")
    loop = GLib.MainLoop()
    loop.run()


if __name__ == "__main__":
    cli()
