#include "script_component.hpp"

params ["_player", "_vehicle"];

if ((_vehicle getVariable [QGVAR(plate), ""]) == "") exitWith {
    false
};
private _discord = _player getVariable [QEGVAR(discord,id), ""];
if (_discord == "") exitWith {};

if (getNumber (missionConfigFile >> "synixe_template") < 3) then {
    private _spawn = switch (true) do {
        case (_vehicle isKindOf "Ship"): { "spawn_sea" };
        case (_vehicle isKindOf "Helicopter"): { "spawn_heli" };
        case (_vehicle isKindOf "Plane"): { "spawn_plane" };
        case (_vehicle isKindOf "Thing"): { "spawn_thing" };
        default { "spawn_land" };
    };

    getPos _vehicle distance getMarkerPos _spawn < 12
} else {
    private _objType = switch (true) do {
        case (_vehicle isKindOf "Ship"): { "sea" };
        case (_vehicle isKindOf "Helicopter"): { "heli" };
        case (_vehicle isKindOf "Plane"): { "plane" };
        case (_vehicle isKindOf "Thing"): { "thing" };
        default { "land" };
    };


    private _spawns = nearestObjects [getPos ,
        SPAWN_TYPES select { _objType in _x },
        100
    ];

    _spawns findIf {
        private _size = getNumber (configOf _x >> QGVAR(size));
        getPos _vehicle distance _x < (_size * 1.5)
    } != -1
}
