
# Messages

CExec has space for 256 different messages, all of which will be documented
(and updated) in this document (not all ADRs are like this, but this one
is authoritative).

# `0x00` - "NOP"

A message packet beginning with `0x00` is a no-operation packet.
It may be arbitrarily long with arbitrary contents, and servers+clients
MAY read as much as they want.

The purpose of this packet is to allow UDP clients to prove they own some
amount of bandwidth to make CExec unable to participate in reflection/amplification
DoS attacks.

A client who sends a 1kb request for 10kb of information over UDP will only receive
the first 1kb of reply data. To receive more, they must send `0x00` packets and
the server will reply byte-for-byte. If an attacker spoofs a UDP source address
they must continue to send data for the CExec server to flood the target, negating
the entire advantage of using CExec as an amplification victim.

If the client is connecting to another server using N+1 forwarding,
the intermediate server (N) __MUST__ forward the NOP packet exactly byte-for-byte.
This will allow the NOP packet to become a protocol-of-last-resort if
we need to do further authentication or if a 3rd-party company wants to send custom
data for some novel, as-of-yet-unknown purpose.

## Example packet

```
+----------+--------------------------+
| 8 bits   |  N bits                  |
+----------+--------------------------+
| type     |  arbitrary values        |
+----------+--------------------------+
| 0x00     | 0x00 0x00 0x01 0xff 0xf0 |
+----------+--------------------------+
```


# `0x01` - "Peer identity record"

A peer identity record is both a request and an announcement. If a client
sends a server it's peer identity, the server __MUST__ respond with it's own
identity. If a server multicasts it's peer identity, clients __SHOULD__ respond
with their peer identity.

The purpose of this packet is to facilitate server identification. If you walk into a
coffee bar and want to know who's there, multicast your own peer identity record (PIR henceforth)
and wait for everyone else to announce themselves.

Peers __MUST__ disregard messages which do not have a matching signature for their name.

Clients __MUST__ differentiate 2 matching names with different public keys (possibly by putting the pub key digest after the given name)


High-level structure:

 - Public key: 0-65535 bytes, utf-8, _usually_ a shielded PGP key block (`-----BEGIN PGP PUBLIC KEY BLOCK-----\n...`)
               but we may end up defining a `FORMAT,Base64==` encoding later depending on what crypto primitives make sense.
 
 - Name: 0-256 bytes, utf-8, defaults to `$HOST-$USER`
 - Signed name: 0-65535 bytes, utf-8, _usually_ a shielded PGP signed message (`-----BEGIN PGP SIGNED MESSAGE-----\n....`)
 
 - Description: 0-65535 bytes, utf-8, a human-readable description of what this peer does / what this peer wants from other peers.
 - Signed description: 0-65535 bytes, utf-8, _usually_ a shielded PGP signed message (`-----BEGIN PGP SIGNED MESSAGE-----\n....`)


## Example packet

```
+---------+----------------+------------------+----------+-------------+
| 8 bits  | 2 bytes        | len bytes        | 8 bits   | len bytes   |
+---------+----------------+------------------+----------+-------------+
| type    | public key len | public key       | name len | name value  |   TODO remaining len + value pairs
+---------+----------------+------------------+----------+-------------+
| 0x01    | 0x06 0xe1      | "-----BEGIN PGP" | 0x09     | "John Snow" |
+---------+----------------+------------------+----------+--------------
```

# `0x02` - "WASM exec request"

Peers may send eachother binary WASM blobs to be executed within a sandbox.

The WASM blob __SHOULD__ have a function named `cexec_main` which takes no arguments and returns an `i32` status code.

If it does not then the function `main` is looked up and executed, with any required arguments being passed in as `0x00` for their type signature. By default a 2-second delay is added before the execution of these mis-built WASM blobs and a warning will be printed to the server's `stderr`.








