# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc
import warnings

import elkr_pb2 as elkr__pb2

GRPC_GENERATED_VERSION = '1.71.0'
GRPC_VERSION = grpc.__version__
_version_not_supported = False

try:
    from grpc._utilities import first_version_is_lower
    _version_not_supported = first_version_is_lower(GRPC_VERSION, GRPC_GENERATED_VERSION)
except ImportError:
    _version_not_supported = True

if _version_not_supported:
    raise RuntimeError(
        f'The grpc package installed is at version {GRPC_VERSION},'
        + f' but the generated code in elkr_pb2_grpc.py depends on'
        + f' grpcio>={GRPC_GENERATED_VERSION}.'
        + f' Please upgrade your grpc module to grpcio>={GRPC_GENERATED_VERSION}'
        + f' or downgrade your generated code using grpcio-tools<={GRPC_VERSION}.'
    )


class ElkrServiceStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.AddSink = channel.unary_unary(
                '/elkr.ElkrService/AddSink',
                request_serializer=elkr__pb2.AddSinkRequest.SerializeToString,
                response_deserializer=elkr__pb2.AddSinkResponse.FromString,
                _registered_method=True)
        self.RemoveSink = channel.unary_unary(
                '/elkr.ElkrService/RemoveSink',
                request_serializer=elkr__pb2.RemoveSinkRequest.SerializeToString,
                response_deserializer=elkr__pb2.RemoveSinkResponse.FromString,
                _registered_method=True)
        self.ListSinks = channel.unary_unary(
                '/elkr.ElkrService/ListSinks',
                request_serializer=elkr__pb2.ListSinksRequest.SerializeToString,
                response_deserializer=elkr__pb2.ListSinksResponse.FromString,
                _registered_method=True)


class ElkrServiceServicer(object):
    """Missing associated documentation comment in .proto file."""

    def AddSink(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def RemoveSink(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def ListSinks(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_ElkrServiceServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'AddSink': grpc.unary_unary_rpc_method_handler(
                    servicer.AddSink,
                    request_deserializer=elkr__pb2.AddSinkRequest.FromString,
                    response_serializer=elkr__pb2.AddSinkResponse.SerializeToString,
            ),
            'RemoveSink': grpc.unary_unary_rpc_method_handler(
                    servicer.RemoveSink,
                    request_deserializer=elkr__pb2.RemoveSinkRequest.FromString,
                    response_serializer=elkr__pb2.RemoveSinkResponse.SerializeToString,
            ),
            'ListSinks': grpc.unary_unary_rpc_method_handler(
                    servicer.ListSinks,
                    request_deserializer=elkr__pb2.ListSinksRequest.FromString,
                    response_serializer=elkr__pb2.ListSinksResponse.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'elkr.ElkrService', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))
    server.add_registered_method_handlers('elkr.ElkrService', rpc_method_handlers)


 # This class is part of an EXPERIMENTAL API.
class ElkrService(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def AddSink(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/elkr.ElkrService/AddSink',
            elkr__pb2.AddSinkRequest.SerializeToString,
            elkr__pb2.AddSinkResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def RemoveSink(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/elkr.ElkrService/RemoveSink',
            elkr__pb2.RemoveSinkRequest.SerializeToString,
            elkr__pb2.RemoveSinkResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)

    @staticmethod
    def ListSinks(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/elkr.ElkrService/ListSinks',
            elkr__pb2.ListSinksRequest.SerializeToString,
            elkr__pb2.ListSinksResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)
