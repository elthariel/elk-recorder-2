import os, time, logging, functools

import grpc

import elkr_pb2_grpc as elkr_grpc
import elkr_pb2 as elkr

log = logging.getLogger(__name__)

WAIT_TIME = 2.0
MEDIA_FILE_FMT = "elkr_{:04}.weba"


class Manager:
    @classmethod
    def wait_for_channel(self, addr, max_tries=23):
        tries = 0
        log.info(f"Checking availability of engine endpoint at: {addr}")

        while tries < max_tries:
            tries += 1

            with grpc.insecure_channel(addr) as channel:
                try:
                    stub = elkr_grpc.ElkrServiceStub(channel)
                    response = stub.ListSinks(elkr.ListSinksRequest())
                    log.info("Engine is available")
                    return True
                except grpc.RpcError as err:
                    if err.code() == grpc.StatusCode.UNAVAILABLE:
                        log.info(f"Engine is unavailable (try {tries}/{max_tries})")
                    else:
                        log.info(
                            f"Engine not available (try {tries}/{max_tries}): error = {err}"
                        )

            time.sleep(WAIT_TIME)

    def __init__(self, address, folder="elkr"):
        self.folder = "elkr"
        self.address = address

        self.sinks = {}

        self.init_sinks()

    def svc(self, channel):
        return elkr_grpc.ElkrServiceStub(channel)

    @property
    def channel(self):
        return grpc.insecure_channel(self.address)

    def init_sinks(self):
        with self.channel as chan:
            response = self.svc(chan).ListSinks(elkr.ListSinksRequest())

        if response.code != elkr.Code.OK:
            raise "Error fetching sinks"

        for sink in response.sinks:
            log.info("Found sink: {sink}")
            mount = self.sink_to_mount(sink)
            self.sinks[mount] = sink

    def sink_to_mount(self, sink):
        elkr_folder = os.path.dirname(sink)
        mount = os.path.dirname(elkr_folder)

        if os.path.basename(elkr_folder) != self.folder:
            log.error(
                "Sink folder ({elkr_folder}) doesn't match the configured folder ({self.folder})"
            )

        return mount

    def sink_to_media(self, sink):
        return os.path.basename(sink)

    def find_next_media(self, mount):
        for idx in range(1_000):
            fname = MEDIA_FILE_FMT.format(idx)
            path = os.path.join(mount, self.folder, fname)

            if not os.path.exists(path):
                return path

        log.error(f"Cannot find available media name for {mount}")

        return None

    def on_mount_add(self, device, mount):
        if mount in self.sinks:
            log.error(f"Mount {mount} already has a sink: {self.sinks[mount]}")
            return

        folder = os.path.join(mount, self.folder)
        if not os.path.exists(folder):
            os.mkdir(folder)

        sink = self.find_next_media(mount)
        if sink is None:
            return

        with self.channel as chan:
            request = elkr.AddSinkRequest(path=sink)
            response = self.svc(chan).AddSink(request)

        if response.code != elkr.Code.OK:
            log.error(f"Unable to add sink: {response}")
            return

        self.sinks[mount] = sink

    def on_mount_remove(self, device, mount):
        if mount not in self.sinks:
            log.warn(f"Mount removal for unknown mount {mount}")
            return

        sink = self.sinks.pop(mount)
        with self.channel as chan:
            request = elkr.RemoveSinkRequest(path=sink)
            response = self.svc(chan).RemoveSink(request)

        log.info(f"Removed sink {sink}")
