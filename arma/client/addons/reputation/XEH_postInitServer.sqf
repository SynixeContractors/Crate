#include "script_component.hpp"

#define TIMEOUT(EV) if (_target getVariable [format ["%1:%2",QGVAR(timeout),EV], 0] > CBA_missionTime) exitWith {}; \
_target setVariable [format ["%1:%2",QGVAR(timeout),EV], CBA_missionTime + 5, true]

[QGVAR(friendly_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("friendly_shot");
    EXTCALL("reputation:friendly_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(civilian_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("civilian_shot");
    EXTCALL("reputation:civilian_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(unarmed_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("unarmed_shot");
    EXTCALL("reputation:unarmed_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(surrendering_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("surrendering_shot");
    EXTCALL("reputation:surrendering_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(captive_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("captive_shot");
    EXTCALL("reputation:captive_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(unconscious_shot), {
    params ["_member", "_target", "_weapon"];
    TIMEOUT("unconscious_shot");
    EXTCALL("reputation:unconscious_shot", [ARR_3(_member, name _target, _weapon)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(friendly_healed), {
    params ["_member", "_target"];
    TIMEOUT("friendly_healed");
    EXTCALL("reputation:friendly_healed", [ARR_2(_member, name _target)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(civilian_healed), {
    params ["_member", "_target"];
    TIMEOUT("civilian_healed");
    EXTCALL("reputation:civilian_healed", [ARR_2(_member, name _target)]);
}] call CBA_fnc_addEventHandler;
