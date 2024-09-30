import { StyleSheet, View } from "react-native";
import React from "react";
import { Button } from "~/components/ui/button";
import { Text } from "~/components/ui/text";
import { router } from "expo-router";
import FontAwesome from "@expo/vector-icons/FontAwesome";
import { useTheme } from "@react-navigation/native";

const index = () => {
  const theme = useTheme();
  const list = [
    {
      name: "Material Purchase",
      link: "/feed-mill/material-purchase",
    },
    { name: "Material Stock", link: "/feed-mill/material-stock" },
    { name: "Feed  Rate", link: "/feed-mill/feed-rate" },
  ];
  return (
    <View style={styles.container}>
      {list.map((item) => (
        <View key={item.name}>
          <Button
            variant={"outline"}
            size={"lg"}
            style={styles.listBtn}
            onPress={() => router.push(item.link)}
          >
            <Text>{item.name}</Text>
            <FontAwesome name="arrow-right" color={theme.colors.text} />
          </Button>
        </View>
      ))}
    </View>
  );
};

export default index;

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: "stretch",
    justifyContent: "center",
    gap: 10,
    paddingHorizontal: 10,
  },
  listBtn: {
    flexDirection: "row",
    justifyContent: "space-between",
  },
});
