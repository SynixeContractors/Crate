#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

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
    EXTCALL("gear:loadout:store",[ARR_2(_discord,str ([_loadout] call FUNC(loadout_clean)))]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:gear") exitWith {};

    switch (_func) do {
        case "loadout:get": {
            (parseSimpleArray _data) params ["_steam", "_loadout"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(loadout_set), [parseSimpleArray _loadout], [_player]] call CBA_fnc_targetEvent;
        };
    };
}];
