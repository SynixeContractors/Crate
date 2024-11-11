#include "script_component.hpp"

params ["_func", "_args"];

switch (_func) do {
    case "load": {
        (parseSimpleArray _data) params ["_id", "_class", "_data"];
        [_id, _class, _data] call FUNC(objects_load);
    };
    case "done": {
        // Handle delayed ACE cargo
        {
            if ((_x getVariable [QGVAR(id), ""]) == "") then { continue };
            _x enableSimulationGlobal true;
            private _cargo = _x getVariable [QGVAR(cargo), []];
            if (_cargo isEqualTo []) then { continue };
            private _object = _x;
            private _loaded = _object getVariable ["ace_cargo_loaded", []];
            if (_loaded isNotEqualTo []) then {
                {
                    if (_x isEqualType objNull) then {
                        detach _x;
                        deleteVehicle _x;
                    };
                } forEach _loaded;
            };
            _object setVariable ["ace_cargo_loaded", [], true];
            private _toLoad = _object getVariable [QGVAR(cargo), []];
            if (_toLoad isNotEqualTo []) then {
                {
                    _x params ["_type", "_id"];
                    if (_type == 0) then {
                        [_id, _object] call ace_cargo_fnc_addCargoItem;
                    } else {
                        private _allObjects = allMissionObjects "All";
                        private _object = _allObjects select (_allObjects findIf { _x getVariable [QGVAR(id), ""] isEqualTo _id });
                        [_object, _object, true] call ace_cargo_fnc_loadItem;
                    };
                } forEach _toLoad;
            };
            [_object] call ace_cargo_fnc_validateCargoSpace;
        } forEach (allMissionObjects "All" - allUnits);
        GVAR(objects_ready) = true;
        EXTCALL("campaigns:groups:load",[GVAR(key)]);
    };
};
