class RscControlsGroupNoScrollbars;
class ctrlButton;

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
