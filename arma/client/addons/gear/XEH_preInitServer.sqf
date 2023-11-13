#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

// ============= Bodybag

[QGVAR(bodybag_store), {
    params [
        ["_discord", "", [""]],
        ["_instigator", "", [""]],
        ["_items", createHashMap, [createHashMap]],
        ["_netId", "", [""]]
    ];
    if (_discord == "") exitWith {};
    if (_instigator == "") exitWith {};
    if (_netId == "") exitWith {};
    EXTCALL("gear:bodybag:store",[ARR_4(_discord,_instigator,_items,_netId)]);
}] call CBA_fnc_addEventHandler;

// ============= Loadout

[QGVAR(loadout_get), {
    params [
        ["_discord", "", [""]],
        ["_steam", "", [""]]
    ];
    if (_discord == "") exitWith {};
    if (_steam == "") exitWith {};
    EXTCALL("gear:loadout:get",[ARR_2(_discord,_steam)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_store), {
    if !(GVAR(enabled)) exitWith {};
    if (GVAR(readOnly)) exitWith {};
    params [
        ["_player", objNull, [objNull]],
        ["_loadout", [], [[]]]
    ];
    if (isNull _player) exitWith {};
    if (_loadout isEqualTo []) exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = getPlayerUID _player;
    EXTCALL("gear:loadout:store",[ARR_3(_discord,_steam,str ([_loadout] call FUNC(loadout_clean)))]);
}] call CBA_fnc_addEventHandler;

// ============= Shop

[QGVAR(shop_items), {
    EXTFUNC("gear:shop:items");
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_enter), {
    if !(GVAR(enabled)) exitWith {};
    if (GVAR(readOnly)) exitWith {};
    params [
        ["_player", objNull, [objNull]],
        ["_items", createHashMap, [createHashMap]]
    ];
    if (isNull _player) exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = getPlayerUID _player;
    EXTCALL("gear:shop:enter", [ARR_3(_discord,_steam,_items)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_leave), {
    if !(GVAR(enabled)) exitWith {};
    if (GVAR(readOnly)) exitWith {};
    params [
        ["_player", objNull, [objNull]],
        ["_loadout", [], [[]]],
        ["_items", createHashMap, [createHashMap]]
    ];
    if (isNull _player) exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = getPlayerUID _player;
    EXTCALL("gear:shop:leave", [ARR_4(_discord,_steam,str ([_loadout] call FUNC(loadout_clean)),_items)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(shop_purchase), {
    if !(GVAR(enabled)) exitWith {};
    if (GVAR(readOnly)) exitWith {};
    params [
        ["_player", objNull, [objNull]],
        ["_items", createHashMap, [createHashMap]]
    ];
    if (isNull _player) exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = getPlayerUID _player;
    EXTCALL("gear:shop:purchase", [ARR_3(_discord,_steam,_items)]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    switch (_name) do {
        case "crate:gear:bodybag": {
            [_func, _data] call FUNC(bodybag_ext);
        };
        case "crate:gear:loadout": {
            [_func, _data] call FUNC(loadout_ext);
        };
        case "crate:gear:shop": {
            [_func, _data] call FUNC(shop_ext);
        };
    };
}];
