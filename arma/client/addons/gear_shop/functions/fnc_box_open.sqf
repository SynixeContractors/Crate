#include "script_component.hpp"

params ["_box"];

// Blur screen
GVAR(blurHandle) = ppEffectCreate ["DynamicBlur", 800];
GVAR(blurHandle) ppEffectEnable true;
GVAR(blurHandle) ppEffectAdjust [8];
GVAR(blurHandle) ppEffectCommit 0.25;

player enableSimulation false;

GVAR(loaders) = createHashMap;
GVAR(loading) = true;
GVAR(loadingShop) = _shop;
player setVariable [QGVAR(open), true, true];
[QGVAR(opening)] call CBA_fnc_localEvent;
