#include "script_component.hpp"

params ["_func", "_args"];

switch (_func) do {
    case "get:set": {
        (parseSimpleArray _data) params ["_steam", "_loadout"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(loadout_set), [parseSimpleArray _loadout], [_player]] call CBA_fnc_targetEvent;
    };
    case "get:empty": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(loadout_track), [], [_player]] call CBA_fnc_targetEvent;
    };
    case "get:err": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(loadout_get_err), [], [_player]] call CBA_fnc_targetEvent;
    };
    case "store:ok": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(loadout_stored), [], [_player]] call CBA_fnc_targetEvent;
    };
    case "store:err": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        ERROR_1("Loadout store error for %1",_steam);
        [QGVAR(loadout_store_err), [], [_player]] call CBA_fnc_targetEvent;
    };
};
