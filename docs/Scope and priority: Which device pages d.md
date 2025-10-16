Scope and priority: Which device pages do you want first beyond Storage and Network? Graphics is a common next target.

That sounds good. start with graphics and make a todo list of other pages to implement.

Backend choice:
Do you want me to add a Rust libvirt binding (and which crate do you prefer), or should I define backend traits and keep it mockable for now?

Use an up-to-date and regularly maintained version of a libvirt Rust binding. If you cant find one, we can work on a separate project to implement the necessary APIs.

Do you plan to keep XML-based device building (virtinst-like) or shift to a native Rust model that later serializes to libvirt XML?

Use Rust serde and create the appropriate structures and then serialize/deserialize those

Integration style: Should the Add Hardware dialog run as a standalone Iced window (like the About dialog now) or integrate into a parent Iced application you plan to build?

It needs to be integrated into a parent Iced application. That should be created based on whatever the main definition of the application is in the python source code in virtManager/

XML editor: Do you want an XML override editor pane in the Rust UI (similar to vmmXMLEditor), or only structured forms?

make it so that editing XML launches a default editor if one is set.