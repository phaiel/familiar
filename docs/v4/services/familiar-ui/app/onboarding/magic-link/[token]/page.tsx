"use client"

import React, { useEffect, useState } from "react"
import { useRouter, useParams } from "next/navigation"
import { Loader2, CheckCircle, XCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { useAuthStore } from "@/stores/auth-store"

export default function MagicLinkPage() {
  const router = useRouter()
  const params = useParams()
  const token = params.token as string
  
  const { 
    consumeMagicLink,
    isAuthenticated,
    needsFamily,
    error,
  } = useAuthStore()

  const [status, setStatus] = useState<"loading" | "success" | "error">("loading")

  useEffect(() => {
    if (!token) {
      setStatus("error")
      return
    }

    const consume = async () => {
      try {
        await consumeMagicLink(token)
        setStatus("success")
      } catch {
        setStatus("error")
      }
    }

    consume()
  }, [token])

  // Redirect after successful authentication
  useEffect(() => {
    if (status === "success" && isAuthenticated) {
      const timer = setTimeout(() => {
        if (needsFamily) {
          router.push("/onboarding/create-family")
        } else {
          router.push("/")
        }
      }, 1500)
      
      return () => clearTimeout(timer)
    }
  }, [status, isAuthenticated, needsFamily, router])

  if (status === "loading") {
    return (
      <Card>
        <CardHeader className="text-center">
          <div className="mx-auto w-12 h-12 rounded-full bg-muted flex items-center justify-center mb-2">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
          <CardTitle>Verifying magic link...</CardTitle>
          <CardDescription>Please wait while we sign you in</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  if (status === "error") {
    return (
      <Card>
        <CardHeader className="text-center">
          <div className="mx-auto w-12 h-12 rounded-full bg-destructive/10 flex items-center justify-center mb-2">
            <XCircle className="h-6 w-6 text-destructive" />
          </div>
          <CardTitle>Invalid magic link</CardTitle>
          <CardDescription>
            {error || "This magic link is invalid, expired, or has already been used."}
          </CardDescription>
        </CardHeader>
        <CardFooter className="flex flex-col gap-2">
          <Button className="w-full" onClick={() => router.push("/onboarding/login")}>
            Try signing in again
          </Button>
          <Button variant="outline" className="w-full" onClick={() => router.push("/onboarding")}>
            Back to onboarding
          </Button>
        </CardFooter>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader className="text-center">
        <div className="mx-auto w-12 h-12 rounded-full bg-green-100 dark:bg-green-900 flex items-center justify-center mb-2">
          <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
        </div>
        <CardTitle>You&apos;re signed in!</CardTitle>
        <CardDescription>Redirecting you to Familiar...</CardDescription>
      </CardHeader>
      <CardContent className="flex justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </CardContent>
    </Card>
  )
}

