#include "script_component.hpp"

params ["_name", "_data"];

private _data = createHashMapFromArray _data;

private _marker = createMarkerLocal [_name, _data getOrDefault ["pos", [0,0,0]]];
_marker setMarkerTypeLocal "hd_dot";
_marker setMarkerColorLocal "ColorBlack";
_marker setMarkerShadowLocal true;

GVAR(markers_ids) pushBackUnique _marker;

[_marker, _data] call EFUNC(common,markerState_load);

[_marker] call zen_area_markers_fnc_onMarkerCreated;
if (_marker select [0,17] == "zen_area_markers_") then {
    private _next = parseNumber (_marker select [17,999]) + 1;
    zen_area_markers_nextId = _next max zen_area_markers_nextId;
};
