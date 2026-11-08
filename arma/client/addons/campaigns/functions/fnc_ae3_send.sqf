#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Sends an AE3 state to the database. Sent in chunks so a large filesystem stays under the callExtension
 * size limit.
 *
 * Arguments:
 * 0: _id <STRING> - Crate ID of the device
 * 1: _class <STRING> - Class of the device
 * 2: _state <ARRAY> - [kind, encoded values] as built by ae3_state
 * 3: _itemClass <STRING> (Optional, default: "") - Inventory item class the device is packed as, if any
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_id", "_class", "_state", ["_itemClass", ""]];

private _text = str _state;
private _size = 4000;
private _offset = 0;
private _chunk = _text select [0, _size];
while {_chunk != ""} do {
    EXTCALL("campaigns:ae3:part",[ARR_2(_id,_chunk)]);
    _offset = _offset + _size;
    _chunk = _text select [_offset, _size];
};
EXTCALL("campaigns:ae3:save",[ARR_3(_id,_class,_itemClass)]);
