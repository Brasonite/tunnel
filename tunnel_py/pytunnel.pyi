from collections.abc import Callable

class PublicKeyParseError(Exception): ...
class TunnelCreationError(Exception): ...
class TunnelDestroyedError(Exception): ...
class TunnelSendingError(Exception): ...

class PublicKey:
    def __init__(self, value: str) -> None:
        """
        Raises:
            PublicKeyParseError: If the value is not a valid public key.
        """
        ...

class Tunnel:
    def __init__(self, handler: Callable) -> None:
        """
        Creates a new Tunnel using the provided handler.

        Args:
            `handler`: The callback which will be called when the Tunnel receives data.

        Raises:
            `TunnelCreationError`: If there was a problem creating the Tunnel.
        """
        ...

    def send(self, address: PublicKey, data: bytes) -> None:
        """
        Sends some data to another tunnel, given the provided address is valid.

        **Note:** if a tunnel is not currently connected to the receiver, it will first attempt to estabilish a connection.

        Args:
            `address`: The **receiver address** of the tunnel to send data to. Can be any value which can be converted to a [PublicKey].
            `data`: The data to be sent. This data can be anything representable as a slice of bytes.

        Raises:
            `TunnelDestroyedError`: If the tunnel was previously destroyed.
            `TunnelSendingError`: If there was a problem sending the data.
        """
        ...

    def destroy(self) -> None:
        """
        Closes both the sender and the receiver endpoint and consumes this object.

        Ideally, this should be called before the execution of the program ends or before a tunnel is discarded.

        **Note:** a tunnel **cannot** be used after this function is called. Using any of a tunnel's functionality whatsoever will raise a `TunnelDestroyedError` when that happens.
        """
        ...

    def close(self, address: PublicKey) -> None:
        """
        Closes a connection to another tunnel, if it exists.

        Args:
            `address`: The address of the other tunnel.
        """
        ...

    def close_all(self) -> None:
        """
        Closes all connections between this tunnel and other tunnels.
        """
        ...

    def sender_address(self) -> PublicKey:
        """
        Returns the address of the sender endpoint of this tunnel.

        The sender enpoint is responsible for sending data to other tunnels. As such, when sending data, this address will be cited as the source.
        """
        ...

    def receiver_address(self) -> PublicKey:
        """
        Returns the address of the receiver endpoint of this tunnel.

        The receiver enpoint is responsible for receiving data from other tunnels. As such, senders should send data to this address.
        """
        ...
