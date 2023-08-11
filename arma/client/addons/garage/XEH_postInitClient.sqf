#include "script_component.hpp"

{
    {
        hideObject _x;
    } forEach allMissionObjects _x;
} forEach SPAWN_TYPES;
