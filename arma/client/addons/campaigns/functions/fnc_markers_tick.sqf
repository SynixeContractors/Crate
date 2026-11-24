#include "..\script_component.hpp"

if !(GVAR(markers_ready)) exitWith {};

if (GVAR(markers_stack) isEqualTo []) then {
    GVAR(markers_stack) = allMapMarkers;
    {
        EXTCALL("campaigns:markers:delete",[ARR_2(GVAR(key),_x)]);
        GVAR(markers_ids) = GVAR(markers_ids) - [_x];
        GVAR(markers_last) deleteAt _x;
    } forEach GVAR(markers_notSeen);
    GVAR(markers_notSeen) = +GVAR(markers_ids);
};
if (GVAR(markers_stack) isEqualTo []) exitWith {};

private _marker = GVAR(markers_stack) deleteAt 0;

// Deleted since the stack was built, it's left unseen so the next sweep deletes it
if (markerShape _marker isEqualTo "ERROR") exitWith {};
// Friendly tracker markers are recreated by their mod, so they're never tracked
if (_marker select [0, 21] == "afft_friendly_tracker") exitWith {};

GVAR(markers_ids) pushBackUnique _marker;
GVAR(markers_notSeen) = GVAR(markers_notSeen) - [_marker];

private _state = [_marker] call EFUNC(common,markerState_save);

_state set ["pos", (getMarkerPos _marker) call EFUNC(common,floatsToStr)];

if (_state isEqualTo (GVAR(markers_last) getOrDefault [_marker, createHashMap])) exitWith {};
GVAR(markers_last) set [_marker, _state];

EXTCALL("campaigns:markers:save",[ARR_3(GVAR(key),_marker,_state)]);
