import { KeyboardIcon, PersonIcon } from "@radix-ui/react-icons";
import { LogOut } from "lucide-react";

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@flow/components";
import { useMeQuery } from "@flow/lib/api";
import { useAuth } from "@flow/lib/auth";
// import { useClient } from "@flow/lib/gql";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

const UserNavigation: React.FC = () => {
  const t = useT();
  const [, setDialogType] = useDialogType();
  const { logout: handleLogout, user } = useAuth();
  // const client = useClient();
  // const [me, setMe] = useState();

  // The normal way
  const getMe = useMeQuery();

  console.log("DATA USER: ", getMe.data);

  // Using plugin typescript-react-query
  // Doesn't work because `Bad argument type. Starting with v5, only the "Object" form is allowed when calling query related functions.`
  // const getMeQuery = useGetMeQuery(client);

  // Using plugin typescript-graphql-request
  // const sdk = getSdk(client);

  // useEffect(() => {
  //   if (me != undefined) return;
  //   (async () => {
  //     const { data } = await sdk.GetMe();
  //     setMe(data);
  //     console.log(data);
  //   })();
  // }, [me, sdk, setMe]);

  const data = { me: { name: "User1111" } };

  // const data = {};

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className="flex gap-2 mr-2">
          <Avatar className="h-8 w-8">
            <AvatarImage src={user?.picture} />
            <AvatarFallback>
              {data?.me?.name ? data?.me.name.charAt(0).toUpperCase() : "F"}
            </AvatarFallback>
          </Avatar>
          <div className="self-center">
            <p className="text-zinc-400 text-sm font-extralight max-w-28 truncate transition-all delay-0 duration-500 hover:max-w-[30vw] hover:delay-500">
              {data?.me?.name ? data?.me.name : "User"}
            </p>
          </div>
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="text-zinc-300 w-[200px]"
        side="bottom"
        align="end"
        sideOffset={4}>
        {/* <DropdownMenuLabel>My Account</DropdownMenuLabel> */}
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("account-settings")}>
          <PersonIcon />
          <p>{t("Account settings")}</p>
        </DropdownMenuItem>
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("keyboard-instructions")}>
          <KeyboardIcon />
          <p>{t("Keyboard shortcuts")}</p>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={handleLogout} className="gap-2">
          <LogOut className="w-[15px] h-[15px] stroke-1" />
          <p>{t("Log out")}</p>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { UserNavigation };
