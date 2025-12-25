# What Should Not Be Done by `NetConf::apply()`

The following actions should be handled by the nispor crate user rather than
nispor crate:

* **Interface Deletion Requirements**: Actions that require interface deletion,
  such as changing VLAN IDs which necessitate deleting the VLAN interface
  first, will not be performed by nispor. Nispor will only delete interfaces
  when explicitly requested.

* **Cross-Interface Modifications**: Actions that require changes to related
  interfaces, such as modifying bond port MTU which typically requires
  corresponding bond MTU adjustments, will not be handled by nispor crate.

* **Overlapping Properties**: Properties that overlap between different
  interface types, such as bond's `ports` property and `BaseInterface`'s
  `controller` property, are not supported in apply operations. To avoid
  conflicts, nispor will not introduce `ports` properties for
  bond, bridge, or other interfaces during apply operations.

* **Interface Ordering**: When creating related interfaces (e.g., a VLAN
  interface and its parent dummy interface within the same `NetConf`), nispor
  does not perform automatic ordering. Users must ensure proper interface
  creation order before invoking `NetConf::apply()`.

* **Complex String-to-Data Type Conversion**: Nispor does not include
  sophisticated deserialization logic to handle quoted string representations
  of boolean values (e.g., `"yes"` or `"true"`) or quoted integer strings.
  Users should preprocess these values before calling `NetConf::apply()`.

* **Userspace Modifications**: Nispor is designed to modify only the Linux
  kernel without touching userspace data. Currently, nispor communicates with
  the kernel through netlink, ioctl, sysfs, proc, and `/run/netns` interfaces.
  Userspace modifications, such as changes to `/etc/resolv.conf`, should be
  handled by the nispor user rather than nispor crate.
