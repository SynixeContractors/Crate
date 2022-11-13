#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[QCVAR(updatedId), {
    call FUNC(diaryEntry);
}] call CBA_fnc_addEventHandler;

[QCVAR(updatedRoles), {
    call FUNC(diaryEntry);
    call FUNC(setTraits);
}] call CBA_fnc_addEventHandler;

