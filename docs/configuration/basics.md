The following custom syntax is defined to make it easier for you to write your configuration:

```nog
set <key> <value>
enable <key>
disable <key>
```

Any key that expects a `Boolean` can be used with the `enable` and `disable` keywords.

In the following table you can see all of the valid keys, what they do and how to use them.

| Key               | Value   | Description                                                                   |
|-------------------|---------|-------------------------------------------------------------------------------|
| min_height        | Number  | The minimum height a window has to have so that it gets managed automatically |
| min_width         | Number  | The minimum width a window has to have so that it gets managed automatically  |
| launch_on_startup | Boolean | Start when you start your computer                                            |
| work_mode         | Boolean | Start in [work mode]()                                                        |
| use_border        | Boolean | Force managed windows to draw a border. (This can help clarity)               |
| light_theme       | Boolean | Changes how the bar colors get generated to fit light colors                  |
| display_app_bar   | Boolean | Enable the bar                                                                |
| remove_title_bar  | Boolean | Remove the titlebar of managed windows                                        |
| remove_task_bar   | Boolean | Remove the taskbar while the program is running                               |