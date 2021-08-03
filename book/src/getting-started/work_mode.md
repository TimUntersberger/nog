# Work mode

`Work mode` is when nog actively tries to manage windows. 

Nog is intended to be used along Windows 10 instead of completely replacing it, that is why it is possible to "activate" and "deactivate" nog.
People that use Windows for development usually use the same machine for gaming and can't be bothered to switch constantly.
I am part of this group so I wanted to create a window manager that supports this use case.

Nog constantly saves a snapshot of the current workspace/window layout and when reentering the work mode it tries to recreate the layout.
The default state can be configured by setting the [work_mode](../configuration/settings.html) setting. 
You can always leave/enter the work mode by calling the [toggle_work_mode](../api/general.html#toggle_work_mode) function.
