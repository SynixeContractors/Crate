#include "script_component.hpp"

if !(GVAR(markers_ready)) exitWith {};

if (GVAR(markers_stack) isEqualTo []) then {
    GVAR(markers_stack) = allMapMarkers;
    {
        EXTCALL("campaigns:markers:delete", _x);
        GVAR(markers_ids) = GVAR(markers_ids) - [_x];
    } forEach GVAR(markers_notSeen);
    GVAR(markers_notSeen) = +GVAR(markers_ids);
};

private _marker = GVAR(markers_stack) deleteAt 0;
GVAR(markers_ids) pushBackUnique _id;

if (markerShape _marker isEqualTo "ERROR") exitWith {};

GVAR(markers_notSeen) = GVAR(markers_notSeen) - [_id];

private _state = [_marker] call EFUNC(common,markerState_save);

EXTCALL("campaigns:groups:save", [ARR_3(GVAR(key), _marker, _state)]);
