#include "script_component.hpp"

params ["_vehicle", "_ammo"];

{
    !(_x getVariable [QGVAR(claimed), false])
} count (nearestObjects [getPos _vehicle, [_ammo], 5]) > 0
