#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

GVAR(objects_ready) = false;
GVAR(objects_stack) = [];
GVAR(objects_ids) = [];
GVAR(objects_notSeen) = [];
GVAR(ae3_pending) = [];
GVAR(ae3_restores) = createHashMap;
GVAR(ae3_parts) = createHashMap;
GVAR(ae3_states) = createHashMap;
GVAR(ae3_packed_known) = createHashMap;

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
        case "crate:campaigns:ae3": {
            [_func, _data] call FUNC(ae3_ext);
        };
    }
}];

addMissionEventHandler ["BuildingChanged", {
    params ["_previousObject", "_newObject", "_isRuin"];
    _newObject setVariable [QEGVAR(common,terrain), netId _previousObject];
}];

["ace_tagCreated", {
    params ["_tag", "_texture", "_object", "_unit"];
    EXTFUNC("uuid");
    private _id = _ext_res select 0;
    private _state = createHashMap;
    _state set ["pos", getPosASL _tag];
    _state set ["rot", [
        vectorDir _tag,
        vectorUp _tag
    ]];
    _state set ["tex", [_texture]];
    EXTCALL("campaigns:objects:save",[ARR_4(GVAR(key),_id,typeOf _tag,_state)]);
}] call CBA_fnc_addEventHandler;
