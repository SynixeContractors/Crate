#include "script_component.hpp"

[QGVAR(updatedId), {
    call FUNC(diaryEntry);
}] call CBA_fnc_addEventHandler;

[QGVAR(updatedRoles), {
    call FUNC(diaryEntry);
}] call CBA_fnc_addEventHandler;
