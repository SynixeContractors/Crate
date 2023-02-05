#include "script_component.hpp"

params ["_player", "_vehicle"];

if !(_vehicle getVariable [QUOTE(ADDON), false]) exitWith {
    false
};
private _discord = _player getVariable [QEGVAR(discord,id), ""];
if (_discord == "") exitWith {};

private _type = switch (true) do {
    case (_vehicle isKindOf "Ship"): { "sea" };
    case (_vehicle isKindOf "Helicopter"): { "heli" };
    case (_vehicle isKindOf "Plane"): { "plane" };
    case (_vehicle isKindOf "Thing"): { "thing" };
    default { "land" };
};
private _spawn = getMarkerPos (switch (_type) do {
    case "sea": { "spawn_sea" };
    case "heli": { "spawn_heli" };
    case "plane": { "spawn_plane" };
    case "thing": { "spawn_thing" };
    default { "spawn_land" };
});

getPos _vehicle distance _spawn < (SPAWN_SIZE * 2)
