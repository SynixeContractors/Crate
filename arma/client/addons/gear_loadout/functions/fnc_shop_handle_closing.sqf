#include "script_component.hpp"

if !(EGVAR(gear_main,enabled)) exitWith {};
if (EGVAR(gear_main,read_only)) exitWith {};

GVAR(tracking) = true;

[] call FUNC(handle_onChange);
