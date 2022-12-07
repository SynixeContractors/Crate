class Extended_PreStart_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preStart));
    };
};
class Extended_PreInit_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preInit));
        clientInit = QUOTE(call COMPILE_FILE(XEH_preInitClient));
        serverInit = QUOTE(call COMPILE_FILE(XEH_preInitServer));
    };
};
class Extended_PostInit_EventHandlers {
    class ADDON {
        clientInit = QUOTE(call COMPILE_FILE(XEH_postInitClient));
        serverInit = QUOTE(call COMPILE_FILE(XEH_postInitServer));
    };
};
class Extended_InventoryOpened_EventHandlers {
    class CAManBase {
        class GVAR(backpackLockInShop) {
            clientInventoryOpened = QUOTE(if (_this select 0 == ACE_player) then {_this call FUNC(handle_onInventoryOpen)});
        };
    };
};
