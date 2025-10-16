# Add Hardware Port TODO

This tracks the remaining device pages and tasks for the Iced-based Add Hardware dialog port.

## Pages to implement (in order)

1. Controller
2. Input
3. Sound
4. Host Device (USB/PCI/MDEV)
5. Char (Serial/Parallel/Console/Channel)
6. Video
7. Watchdog
8. Filesystem
9. Smartcard
10. USB Redirection
11. TPM
12. RNG
13. Panic
14. Vsock

## Cross-cutting tasks

- Backend integration: adopt a maintained Rust libvirt binding; if not available, define traits and provide an adapter crate.
- Device models: create serde structs for each device type (XML parity) and implement to/from XML (quick-xml).
- Domain capabilities: fetch and plumb recommended values (buses, models, etc.) into UI pick lists.
- Async/progress: replace Python vmmAsyncJob with tokio-based tasks and an Iced progress modal.
- XML editing: Support launching default editor (VISUAL/EDITOR) for each device's XML.
- Parent app integration: Embed Add Hardware view into the main Iced application replacing the standalone window.
- Validation: Port validation rules from virtinst and Python helpers.
- Storage browser: Implement a cross-platform file/pool browser for images/volumes.
- Internationalization: add gettext/translation strategy for UI labels.
