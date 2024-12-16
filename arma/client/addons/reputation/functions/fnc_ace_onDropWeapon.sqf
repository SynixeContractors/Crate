#include "script_component.hpp"

params ["_unit"];

_unit setVariable [QGVAR(unarmedDelay), CBA_missionTime + 3, true];
