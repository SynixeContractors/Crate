#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(EGVAR(gear_main,enabled)) exitWith {};

EXTFUNC("gear:shop:items");
