#include "script_component.hpp"

FUNC(setSpawnHidden) = {
    params ["_hide"];
    {
        {
            _x hideObject _hide;
        } forEach allMissionObjects _x;
    } forEach SPAWN_TYPES;
};

["zen_curatorDisplayLoaded", {
    [false] call FUNC(setSpawnHidden);
}] call CBA_fnc_addEventHandler;

["zen_curatorDisplayUnloaded", {
    [true] call FUNC(setSpawnHidden);
}] call CBA_fnc_addEventHandler;

[true] call FUNC(setSpawnHidden);
