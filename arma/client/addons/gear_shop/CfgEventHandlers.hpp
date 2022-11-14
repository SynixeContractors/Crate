class Extended_PreStart_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preStart));
    };
};
class Extended_PreInit_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preInit));
        initClient = QUOTE(call COMPILE_FILE(XEH_preInitCient));
        initServer = QUOTE(call COMPILE_FILE(XEH_preInitServer));
    };
};
class Extended_PostInit_EventHandlers {
    class ADDON {
        initClient = QUOTE(call COMPILE_FILE(XEH_postInitCient));
        initServer = QUOTE(call COMPILE_FILE(XEH_postInitServer));
    };
};
class Extended_InventoryOpened_EventHandlers {
    class CAManBase {
        class GVAR(backpackLockInShop) {
            clientInventoryOpened = QUOTE(if (_this select 0 == ACE_player) then {_this call FUNC(onInventoryOpen)});
        };
    };
};
