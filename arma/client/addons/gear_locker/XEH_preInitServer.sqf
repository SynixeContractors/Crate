#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

GVAR(importing) = createHashMap;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:gear:locker") exitWith {};

    switch (_func) do {
        case "get:clear": {
            params ["_steam"];
            GVAR(importing) set [_steam, createHashMap];
        };
        case "get:add": {
            (parseSimpleArray _data) params ["_steam", "_class", "_count"];
            GVAR(importing) getOrDefault [_steam, createHashMap] set [_class, _count];
        };
        case "get:done": {
            (parseSimpleArray _data) params ["_steam"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(set), [GVAR(importing) getOrDefault [_steam, createHashMap]], [_player]] call CBA_fnc_targetEvent;
        };
        case "store": {
            (parseSimpleArray _data) params ["_steam", "_result"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(stored), [_result], [_player]] call CBA_fnc_targetEvent;
        };
    };
}];
