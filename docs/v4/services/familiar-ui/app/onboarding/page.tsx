"use client"

import React, { useEffect, useState } from "react"
import { useRouter, useSearchParams } from "next/navigation"
import Link from "next/link"
import { ArrowRight, Mail, KeyRound, Users } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useAuthStore, InvitationInfo } from "@/stores/auth-store"

export default function OnboardingPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const inviteCode = searchParams.get("code")
  
  const { 
    isAuthenticated, 
    needsFamily,
    pendingInvitation,
    checkInviteCode,
    isLoading,
    error,
    clearError,
  } = useAuthStore()
  
  const [codeInput, setCodeInput] = useState(inviteCode || "")
  const [checkingCode, setCheckingCode] = useState(false)
  const [codeError, setCodeError] = useState<string | null>(null)

  // Check invite code from URL
  useEffect(() => {
    if (inviteCode && !pendingInvitation) {
      handleCheckCode(inviteCode)
    }
  }, [inviteCode])

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated && !needsFamily) {
      router.push("/")
    } else if (isAuthenticated && needsFamily) {
      router.push("/onboarding/create-family")
    }
  }, [isAuthenticated, needsFamily, router])

  const handleCheckCode = async (code: string) => {
    setCheckingCode(true)
    setCodeError(null)
    
    try {
      await checkInviteCode(code)
    } catch (e) {
      setCodeError(e instanceof Error ? e.message : "Invalid code")
    } finally {
      setCheckingCode(false)
    }
  }

  // Show invitation info if we have a valid code
  if (pendingInvitation) {
    return (
      <Card>
        <CardHeader className="text-center">
          <div className="mx-auto w-12 h-12 rounded-full bg-green-100 dark:bg-green-900 flex items-center justify-center mb-2">
            <Users className="h-6 w-6 text-green-600 dark:text-green-400" />
          </div>
          <CardTitle>You&apos;re invited!</CardTitle>
          <CardDescription>
            Join the <strong>{pendingInvitation.tenant_name}</strong> family
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-lg bg-muted p-4 text-center">
            <p className="text-sm text-muted-foreground">You&apos;ll join as</p>
            <p className="font-medium capitalize">{pendingInvitation.role}</p>
          </div>
          
          {!pendingInvitation.is_valid && (
            <div className="rounded-lg bg-destructive/10 p-4 text-center text-destructive text-sm">
              This invitation has expired or reached its usage limit.
            </div>
          )}
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          {pendingInvitation.is_valid ? (
            <>
              <Button 
                className="w-full" 
                onClick={() => router.push(`/onboarding/signup?code=${inviteCode}`)}
              >
                Create Account & Join
                <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
              <p className="text-xs text-muted-foreground text-center">
                Already have an account?{" "}
                <Link href={`/onboarding/login?code=${inviteCode}`} className="text-primary hover:underline">
                  Sign in
                </Link>
              </p>
            </>
          ) : (
            <Button variant="outline" className="w-full" onClick={() => useAuthStore.getState().setPendingInvitation(null)}>
              Try Another Code
            </Button>
          )}
        </CardFooter>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader className="text-center">
        <CardTitle>Welcome to Familiar</CardTitle>
        <CardDescription>
          Get started by creating an account or joining an existing family
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="new" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="new">New Family</TabsTrigger>
            <TabsTrigger value="join">Join Family</TabsTrigger>
          </TabsList>
          
          <TabsContent value="new" className="space-y-4 mt-4">
            <p className="text-sm text-muted-foreground text-center">
              Create a new family and invite your loved ones to join.
            </p>
            <Button className="w-full" onClick={() => router.push("/onboarding/signup")}>
              Create Account
              <ArrowRight className="ml-2 h-4 w-4" />
            </Button>
          </TabsContent>
          
          <TabsContent value="join" className="space-y-4 mt-4">
            <div className="space-y-2">
              <Label htmlFor="invite-code">Invitation Code</Label>
              <div className="flex gap-2">
                <Input
                  id="invite-code"
                  placeholder="XXXXXXXX"
                  value={codeInput}
                  onChange={(e) => setCodeInput(e.target.value.toUpperCase())}
                  className="font-mono tracking-wider"
                />
                <Button 
                  onClick={() => handleCheckCode(codeInput)}
                  disabled={!codeInput || checkingCode}
                >
                  {checkingCode ? "..." : "Check"}
                </Button>
              </div>
              {codeError && (
                <p className="text-sm text-destructive">{codeError}</p>
              )}
            </div>
            <p className="text-xs text-muted-foreground text-center">
              Ask a family member for an invitation code to join their family.
            </p>
          </TabsContent>
        </Tabs>
      </CardContent>
      <CardFooter className="flex justify-center">
        <p className="text-xs text-muted-foreground">
          Already have an account?{" "}
          <Link href="/onboarding/login" className="text-primary hover:underline">
            Sign in
          </Link>
        </p>
      </CardFooter>
    </Card>
  )
}

