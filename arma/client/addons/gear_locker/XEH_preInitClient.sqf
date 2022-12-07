#include "script_component.hpp"

GVAR(items) = createHashMap;

[QGVAR(set), {
    switch (_this select 0) do {
        case "ok": {
            GVAR(items) = _this select 1;
            [QEGVAR(gear_shop,loader_ready), QUOTE(ADDON)] call CBA_fnc_localEvent;

            ["ace_arsenal_leftPanelFilled", [findDisplay 1127001]] call CBA_fnc_localEvent;
            ["ace_arsenal_rightPanelFilled", [findDisplay 1127001]] call CBA_fnc_localEvent;
        };
        default {
            [QEGVAR(gear_shop,loader_error), QUOTE(ADDON)] call CBA_fnc_localEvent;
        };
    };
}] call CBA_fnc_addEventHandler;

[QEGVAR(gear_shop,opening), FUNC(shop_handle_opening)] call CBA_fnc_addEventHandler;
[QEGVAR(gear_shop,closing), FUNC(shop_handle_closing)] call CBA_fnc_addEventHandler;
[QEGVAR(gear_shop,reverting), FUNC(shop_handle_reverting)] call CBA_fnc_addEventHandler;
