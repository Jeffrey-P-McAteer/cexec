
# CExec overview

CExec is first and foremost an always-on, all-protocols, limited execution engine.

Servers should always be discoverable, and the design is to allow servers to facilitate
N+1 peer discovery and packet forwarding, even when using UDP multicast.

Messaging is similarly brutal; the wire format for cexec messages is:

```
+--------+------------------------------------------+
| 8 bits |  N bits (lengths often precede contents) |
+--------+------------------------------------------+
|  type  | <type-defined structure>                 |
+--------+------------------------------------------+
```

Note there is no plan for any kind of encryption. There are plans to strictly
sign and validate the origin of programs, but their contents are unencrypted.

People desiring encryption should send their peers a program that encrypts further communication,
using whatever auth/pki/protocol that fits your use-case.

# Servers






