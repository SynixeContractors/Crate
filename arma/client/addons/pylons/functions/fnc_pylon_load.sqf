#include "script_component.hpp"

params ["_vehicle", "_ammo", "_index"];

private _nearby = nearestObjects [getPos _vehicle, [_ammo], 5];
private _nearbyIndex = _nearby findIf {
    !(_x getVariable [QGVAR(claimed), false])
};
if (_nearbyIndex == -1) exitWith {};

// TODO progress bar

_vehicle setAmmoOnPylon [_index, (_vehicle ammoOnPylon _index) + 1];
deleteVehicle (_nearby select _nearbyIndex);
