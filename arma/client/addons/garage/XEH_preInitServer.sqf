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
