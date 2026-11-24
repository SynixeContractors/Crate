#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Finishes loading campaign objects once every one exists and has its state applied: loads ACE cargo,
 * enables simulation, applies AE3 state, then starts loading groups.
 *
 * Arguments:
 * None
 *
 * Return Value:
 * None
 *
 * Public: No
 */

private _byId = createHashMap;
{
    private _id = _x getVariable [QGVAR(id), ""];
    if (_id != "") then {
        _byId set [_id, _x];
    };
} forEach (allMissionObjects "All" - allUnits);

{
    private _vehicle = _y;
    _vehicle enableSimulationGlobal true;

    private _toLoad = _vehicle getVariable [QGVAR(cargo), []];
    _vehicle setVariable [QGVAR(cargo), nil];
    if (_toLoad isEqualTo []) then { continue };

    {
        if (_x isEqualType objNull) then {
            detach _x;
            deleteVehicle _x;
        };
    } forEach (_vehicle getVariable ["ace_cargo_loaded", []]);
    _vehicle setVariable ["ace_cargo_loaded", [], true];

    {
        _x params ["_type", "_data"];
        if (_type == 0) then {
            [_data, _vehicle] call ace_cargo_fnc_addCargoItem;
        } else {
            if (_data isEqualType []) then {
                // Saved before cargo objects were referenced by ID: [class, state]
                _data params ["_class", "_state"];
                private _item = _class createVehicle [0,0,0];
                [_item, _state] call EFUNC(common,objectState_load);
                [_item, _vehicle, true] call ace_cargo_fnc_loadItem;
            } else {
                private _item = _byId getOrDefault [_data, objNull];
                if (isNull _item) then {
                    WARNING_2("Cargo object %1 of %2 was not found",_data,_vehicle);
                } else {
                    [_item, _vehicle, true] call ace_cargo_fnc_loadItem;
                };
            };
        };
    } forEach _toLoad;
    [_vehicle] call ace_cargo_fnc_validateCargoSpace;
} forEach _byId;

{
    _x params ["_object", "_state"];
    if (!isNull _object) then {
        [_object, _state] call FUNC(ae3_apply);
    };
} forEach GVAR(ae3_pending);
GVAR(ae3_pending) = [];

GVAR(objects_ready) = true;
EXTCALL("campaigns:groups:load",[GVAR(key)]);
