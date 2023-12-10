#include "script_component.hpp"

if !(isMultiplayer) exitWith {};
if !(GVAR(enabled)) exitWith {};

GVAR(spawned) = createHashMap;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    switch (_name) do {
        case "crate:garage": {
            switch (_func) do {
                case "spawn": {
                    (parseSimpleArray _data) call FUNC(spawn);
                };
                case "store": {
                    (parseSimpleArray _data) params ["_plate"];
                    private _vehicle = GVAR(spawned) deleteAt _plate;
                    deleteVehicle _vehicle;
                }
            };
        };
    };
}];

[QGVAR(store), {
    params ["_player", "_vehicle"];
    private _plate = _vehicle getVariable [QGVAR(plate), ""];
    if (_plate == "") exitWith {};
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    if (_discord == "") exitWith {};

    private _state = [_vehicle] call EFUNC(common,objectState_save);

    EXTCALL("garage:store",[ARR_3(_plate, _state, _discord)]);
}] call CBA_fnc_addEventHandler;
