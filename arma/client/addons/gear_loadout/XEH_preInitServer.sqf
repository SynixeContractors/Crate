#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

if !(isMultiplayer) exitWith {};

[QGVAR(get), {
    params [
        ["_discord", "", [""]],
        ["_steam", "", [""]]
    ];
    if (_discord == "") exitWith {};
    if (_steam == "") exitWith {};
    EXTCALL("gear:loadout:get",[ARR_2(_discord,_steam)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(store), {
    params [
        ["_player", objNull, [objNull]],
        ["_loadout", [], [[]]]
    ];
    if (isNull _player) exitWith {};
    if (_loadout isEqualTo []) exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = getPlayerUID _player;
    EXTCALL("gear:loadout:store",[ARR_3(_discord,_steam,str ([_loadout] call FUNC(clean)))]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:gear:loadout") exitWith {};

    switch (_func) do {
        case "get:set": {
            (parseSimpleArray _data) params ["_steam", "_loadout"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(set), [parseSimpleArray _loadout], [_player]] call CBA_fnc_targetEvent;
            [QGVAR(track), [], [_player]] call CBA_fnc_targetEvent;
        };
        case "get:empty": {
            (parseSimpleArray _data) params ["_steam"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(track), [], [_player]] call CBA_fnc_targetEvent;
        };
        case "store": {
            (parseSimpleArray _data) params ["_steam", "_result"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(stored), [_result], [_player]] call CBA_fnc_targetEvent;
        };
    };
}];
