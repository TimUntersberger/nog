ignore "File Explorer";
ignore "Task Manager";

rule ".*- Mozilla Firefox|Mozilla Firefox" #{
    has_custom_titlebar: true,
    workspace: 2,
    firefox: true,
};

rule ".*- Discord|Discord" #{
    has_custom_titlebar: true,
};

rule "Spotify Premium" #{
    has_custom_titlebar: true,
};

rule ".*- Google Chrome" #{
    has_custom_titlebar: true,
    workspace: 2,
    chromium: true,
};

rule ".*- Visual Studio Code" #{
    has_custom_titlebar: true,
};