import { StyleSheet } from "react-native";
import React from "react";
import { Tabs } from "expo-router";
import { ThemeToggle } from "~/components/ThemeToggle";
import FontAwesome from "@expo/vector-icons/FontAwesome";

const _layout = () => {
  return (
    <Tabs
      screenOptions={{
        headerRight: () => <ThemeToggle />,
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: "Feed Mill",
          tabBarIcon: ({ color }) => (
            <FontAwesome name="building" size={26} color={color} />
          ),
        }}
      ></Tabs.Screen>
      <Tabs.Screen
        name="farm"
        options={{
          title: "Poultry Farm",
          tabBarIcon: ({ color }) => (
            <FontAwesome name="industry" size={26} color={color} />
          ),
        }}
      ></Tabs.Screen>
      <Tabs.Screen
        name="settings"
        options={{
          title: "Settings",
          tabBarIcon: ({ color }) => (
            <FontAwesome name="gear" size={26} color={color} />
          ),
        }}
      ></Tabs.Screen>
    </Tabs>
  );
};

export default _layout;

const styles = StyleSheet.create({});
