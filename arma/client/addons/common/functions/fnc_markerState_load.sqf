#include "script_component.hpp"

params ["_marker", "_state"];

{
    switch (_x) do {
        case "alpha": {
            _marker setMarkerAlphaLocal _y;
        };
        case "color": {
            _marker setMarkerColorLocal _y;
        };
        case "size": {
            _marker setMarkerSizeLocal _y;
        };
        case "text": {
            _marker setMarkerTextLocal _y;
        };
        case "type": {
            _marker setMarkerTypeLocal _y;
        };
        case "brush": {
            _marker setMarkerBrushLocal _y;
        };
        case "dir": {
            _marker setMarkerDirLocal _y;
        };
        case "polyline": {
            _marker setMarkerShapeLocal "POLYLINE";
            _marker setMarkerPolylineLocal _y;
        };
        case "shadow": {
            _marker setMarkerShadowLocal _y;
        };
        case "shape": {
            _marker setMarkerShapeLocal _y;
        };
    };
} forEach _state;

// Broadcast changes
_marker setMarkerAlpha (markerAlpha _marker);
