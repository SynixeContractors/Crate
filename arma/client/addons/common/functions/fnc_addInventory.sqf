#include "..\script_component.hpp"

params ["_container", "_cargo"];

private _standard = _cargo select 0;
private _nested = _cargo select 1;

// Magazine cargo
{
    _container addMagazineCargoGlobal [_x, _standard select 0 select 1 select _forEachIndex];
} forEach ((_standard select 0) select 0);

// Weapon Cargo
{
    _container addWeaponWithAttachmentsCargoGlobal [_x, 1];
} forEach (_standard select 1);

// Item Cargo
{
    _container addItemCargoGlobal [_x, _standard select 2 select 1 select _forEachIndex];
} forEach ((_standard select 2) select 0);

// Nested Cargo
{
    _x params ["_class", "_items"];
    _container addBackpackCargoGlobal [_class, 1];
    // Thanks Bohemia, the only way to find the container I just added is to loop over every container...
    {
        private _found = false;
        if (_x select 0 == _class) then {
            // More Elegant code
            if (
                !_found &&
                { (_x select 1) getVariable [QGVAR(NOEDIT), true] }
            ) then {
                [_x select 1, _items] call FUNC(addInventory);
                (_x select 1) setVariable [QGVAR(NOEDIT), false];
                _found = true;
            };
        };
    } forEach everyContainer _container;
} forEach _nested;
