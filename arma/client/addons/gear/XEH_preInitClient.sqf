#include "script_component.hpp"

if !(hasInterface) exitWith {};
if !(isMultiplayer) exitWith {};

[QGVAR(notify), {
    params ["_message"];
    systemChat _message;
}] call CBA_fnc_addEventHandler;

// ============= Loadout

GVAR(loadout_tracking) = false;

[QGVAR(loadout_set), {
    params ["_loadout"];
    [player, _loadout, false] call CBA_fnc_setLoadout;
    [{
        private _loadout = _this select 0;
        if (count _loadout == 2) then {
            _loadout = _loadout select 0;
        };
        player addGoggles (_loadout select 7);
        GVAR(loadout_tracking) = true;
    }, [_loadout]] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_track), {
    [{
        GVAR(loadout_tracking) = true;
    }] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_get_err), {
    systemChat "Error getting loadout from server, your loadout will not be tracked.";
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_store_err), {
    ERROR("Error storing loadout on server");
}] call CBA_fnc_addEventHandler;

[QEGVAR(discord,updatedId), {
    if !(GVAR(enabled)) exitWith {};
    [QGVAR(loadout_get),[
        player getVariable [QEGVAR(discord,id), ""],
        getPlayerUID player
    ]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

// ============= Shop

GVAR(shop_processing) = false;

["ace_arsenal_displayOpened", FUNC(shop_arsenal_opened)] call CBA_fnc_addEventHandler;
["ace_arsenal_displayClosed", FUNC(shop_arsenal_closed)] call CBA_fnc_addEventHandler;
["ace_arsenal_leftPanelFilled", FUNC(shop_arsenal_leftPanelFilled)] call CBA_fnc_addEventHandler;
["ace_arsenal_rightPanelFilled", FUNC(shop_arsenal_rightPanelFilled)] call CBA_fnc_addEventHandler;
["ace_arsenal_loadoutsListFilled", FUNC(shop_arsenal_loadoutsListFilled)] call CBA_fnc_addEventHandler;

[QGVAR(shop_enter_ok), FUNC(shop_enter)] call CBA_fnc_addEventHandler;
[QGVAR(shop_enter_err), {
    GVAR(shop_blurHandle) ppEffectAdjust [0];
    GVAR(shop_blurHandle) ppEffectCommit 0.25;
    [{
        GVAR(shop_blurHandle) ppEffectEnable false;
        ppEffectDestroy GVAR(shop_blurHandle);
    }, [], 1] call CBA_fnc_waitAndExecute;
    player setVariable [QGVAR(shop_open), false, true];
    player enableSimulation true;
    systemChat "An error occurred while trying to enter the shop, please try again later.";
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_leave_ok), {
    player setVariable [QGVAR(shop_open), false, true];
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_leave_err), {
    systemChat "An error occurred while trying to leave the shop, removing items.";
    [player, [[],[],[],[],[],[],"","",[],["","","","","",""]], false] call CBA_fnc_setLoadout;
    player setVariable [QGVAR(shop_open), false, true];
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_purchase_ok), {
    systemChat "Purchase successful";
    params ["_locker", "_balance"];
    GVAR(shop_locker) = createHashMapFromArray _locker;
    GVAR(shop_balance) = _balance;
    GVAR(shop_processing) = false;
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_purchase_err), {
    systemChat "An error occurred while trying to purchase";
    GVAR(shop_processing) = false;
}] call CBA_fnc_addEventHandler;
