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
        case (_vehicle isKindOf "Ship"): { QGVAR(sea) };
        case (_vehicle isKindOf "Helicopter"): { QGVAR(heli) };
        case (_vehicle isKindOf "Plane"): { QGVAR(plane) };
        case (_vehicle isKindOf "Thing"): { QGVAR(thing) };
        default { QGVAR(land) };
    };

    private _size = getNumber (configFile >> "CfgVehicles" >> _objType >> QGVAR(size));

    (allMissionObjects _objType) findIf {
        getPos _vehicle distance _x < (_size * 2)
    } != -1
}
