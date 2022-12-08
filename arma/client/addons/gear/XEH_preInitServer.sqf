#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

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

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    switch (_name) do {
        case "crate:gear:loadout": {
            [_func, _data] call FUNC(ext_loadout);
        };
    };
}];
