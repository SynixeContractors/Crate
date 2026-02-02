class ctrlButton;
class RscCombo;
class RscControlsGroupNoScrollbars;
class RscListBox;

class ace_arsenal_display {
    class controls {
        class menuBar: RscControlsGroupNoScrollbars {
            class controls {
                class buttonHide: ctrlButton {
                    onButtonClick = QUOTE([ctrlParent (_this select 0)] call FUNC(shop_arsenal_btnHide));
                };
            };
        };
        class rightTabContentListnBox: RscListNBox {
            onLBSelChanged = QUOTE(_this call FUNC(shop_arsenal_rightPanelSelChanged));
        };

        class leftRoleFilter: RscCombo {
            idc = IDC_leftRoleFilter;
            onLBSelChanged = QUOTE(call FUNC(shop_arsenal_roleFilter));
            x = QUOTE(safeZoneX + 13 * GRID_W);
            y = QUOTE(safeZoneY + 14 * GRID_H);
            w = QUOTE(80 * GRID_W);
            h = QUOTE(6 * GRID_H);
            sizeEx = QUOTE(5 * GRID_H);
        };

        class leftTabContent: RscListBox {
            y = QUOTE(safeZoneY + 20 * GRID_H);
            h = QUOTE(safeZoneH - 30.5 * GRID_H);
        };
    };
};

class ctrlControlsGroupNoScrollbars;

class ace_arsenal_loadoutsDisplay {
    class controls {
        class centerBox: ctrlControlsGroupNoScrollbars {
            class controls {
                class contentPanel: RscListNBox {
                    columns[]={0, 0.05, 0.35, 0.45, 0.55, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90};
                };
            };
        };
    };
};
