#include "script_component.hpp"

params ["_marker"];

private _state = createHashMap;

if (markerAlpha _marker != 1) then {
    _state set ["alpha", markerAlpha _marker];
};
if (markerColor _marker != "ColorBlack") then {
    _state set ["color", markerColor _marker];
};
if (markerSize _marker isNotEqualTo [1,1]) then {
    _state set ["size", markerSize _marker];
};
if (markerText _marker != "") then {
    _state set ["text", markerText _marker];
};
if (markerType _marker != "hd_dot") then {
    _state set ["type", markerType _marker];
};
if (markerBrush _marker != "Solid") then {
    _state set ["brush", markerBrush _marker];
};
if (markerDir _marker != 0) then {
    _state set ["dir", markerDir _marker];
};
if (markerPolyline _marker isNotEqualTo []) then {
    _state set ["polyline", markerPolyline _marker];
};
if !(markerShadow _marker) then {
    _state set ["shadow", false];
};
if (markerShape _marker != "ICON") then {
    _state set ["shape", markerShape _marker];
};

_state
