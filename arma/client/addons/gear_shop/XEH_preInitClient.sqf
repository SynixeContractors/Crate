#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

GVAR(boxes) = [];

[QGVAR(loader_register), FUNC(handleLoaderRegister)] call CBA_fnc_addEventHandler;
[QGVAR(loader_ready), FUNC(handleLoaderReady)] call CBA_fnc_addEventHandler;
[QGVAR(loader_error), FUNC(handleLoaderError)] call CBA_fnc_addEventHandler;
