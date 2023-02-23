#include "script_component.hpp"

params ["_name", "_data"];

private _data = createHashMapFromArray _data;

private _marker = createMarkerLocal [_name, _date getOrDefault ["pos", [0,0,0]]];
_marker setMarkerTypeLocal "hd_dot";
_marker setMarkerColorLocal "ColorBlack";
_marker setMarkerShadowLocal true;

GVAR(markers_ids) pushBackUnique _marker;

[_marker, _data] call EFUNC(common,markerState_load);
