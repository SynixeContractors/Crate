#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"

[QGVAR(setDate), {
    setDate _this;
}] call CBA_fnc_addEventHandler;

[QGVAR(introText), {
    [] call FUNC(introText);
}] call CBA_fnc_addEventHandler;

ADDON = true;
