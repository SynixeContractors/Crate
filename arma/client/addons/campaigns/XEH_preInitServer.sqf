#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

GVAR(objects_ready) = false;
GVAR(objects_stack) = [];
GVAR(objects_ids) = [];
GVAR(objects_notSeen) = [];

GVAR(groups_ready) = false;
GVAR(groups_stack) = [];
GVAR(groups_ids) = [];
GVAR(groups_notSeen) = [];

GVAR(units_ready) = false;
GVAR(units_stack) = [];
GVAR(units_ids) = [];
GVAR(units_notSeen) = [];

GVAR(markers_ready) = false;
GVAR(markers_stack) = [];
GVAR(markers_ids) = [];
GVAR(markers_notSeen) = [];

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    switch (_name) do {
        case "crate:campaigns:objects": {
            [_func, _data] call FUNC(objects_ext);
        };
        case "crate:campaigns:groups": {
            [_func, _data] call FUNC(groups_ext);
        };
        case "crate:campaigns:units": {
            [_func, _data] call FUNC(units_ext);
        };
        case "crate:campaigns:markers": {
            [_func, _data] call FUNC(markers_ext);
        };
    }
}];
