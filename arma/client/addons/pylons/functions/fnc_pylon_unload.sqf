#include "script_component.hpp"

params ["_vehicle", "_index", "_ammo"];

private _current = _vehicle ammoOnPylon _index;

if (_current == 0) exitWith {};

private _new = _current - 1;

// TODO progress bar

_vehicle setAmmoOnPylon [_index, _new];
createVehicle [_ammo, getPos _vehicle, [], 0, "NONE"];
