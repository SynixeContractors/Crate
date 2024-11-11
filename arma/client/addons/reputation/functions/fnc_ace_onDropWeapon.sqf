#include "script_component.hpp"

params ["_unit"];

_unit setVariable [QGVAR(unarmedSince), CBA_missionTime, true];
